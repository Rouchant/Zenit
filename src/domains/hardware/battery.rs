use windows_sys::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

pub fn parse_battery_wh_from_xml(xml: &str) -> Option<u32> {
    let mut total_mwh: u64 = 0;

    // First scan <Batteries> block if it exists
    let batteries_block = if let Some(start) = xml.find("<Batteries>") {
        if let Some(end) = xml[start..].find("</Batteries>") {
            &xml[start..start + end]
        } else {
            xml
        }
    } else {
        xml
    };

    let mut cursor = batteries_block;
    while let Some(pos) = cursor.find("<DesignCapacity>") {
        let after = &cursor[pos + 16..];
        if let Some(end_pos) = after.find("</DesignCapacity>") {
            let num_str = after[..end_pos].trim();
            if let Ok(mwh) = num_str.parse::<u64>() {
                total_mwh += mwh;
            }
            cursor = &after[end_pos + 17..];
        } else {
            break;
        }
    }

    if total_mwh > 0 {
        return Some(mwh_to_wh(total_mwh));
    }

    // Fallback: check <RuntimeEstimates><DesignCapacity><Capacity>...
    if let Some(pos) = xml.find("<DesignCapacity") {
        let slice = &xml[pos..];
        if let Some(end_pos) = slice.find("</DesignCapacity>") {
            let inner = &slice[..end_pos];
            if let Some(cap_pos) = inner.find("<Capacity>") {
                let cap_slice = &inner[cap_pos + 10..];
                if let Some(cap_end) = cap_slice.find("</Capacity>") {
                    let num_str = cap_slice[..cap_end].trim();
                    if let Ok(mwh) = num_str.parse::<u64>() {
                        if mwh > 0 {
                            return Some(mwh_to_wh(mwh));
                        }
                    }
                }
            }
        }
    }

    // Fallback: check <FullChargeCapacity>
    if let Some(pos) = xml.find("<FullChargeCapacity>") {
        let after = &xml[pos + 20..];
        if let Some(end_pos) = after.find("</FullChargeCapacity>") {
            let num_str = after[..end_pos].trim();
            if let Ok(mwh) = num_str.parse::<u64>() {
                if mwh > 0 {
                    return Some(mwh_to_wh(mwh));
                }
            }
        }
    }

    None
}

fn mwh_to_wh(val: u64) -> u32 {
    if val >= 200 {
        ((val as f64) / 1000.0).round() as u32
    } else {
        val as u32
    }
}

pub fn detect_battery_capacity_wh() -> Option<u32> {
    let temp_dir = std::env::temp_dir();
    let report_path = temp_dir.join(format!("zenit_bat_{}.xml", std::process::id()));
    let report_str = report_path.to_string_lossy().to_string();

    let mut cmd = std::process::Command::new("powercfg");
    cmd.args(["/batteryreport", "/xml", "/output", &report_str]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    if let Ok(res) = cmd.output() {
        if res.status.success() && report_path.exists() {
            if let Ok(xml) = std::fs::read_to_string(&report_path) {
                let _ = std::fs::remove_file(&report_path);
                if let Some(wh) = parse_battery_wh_from_xml(&xml) {
                    return Some(wh);
                }
            } else {
                let _ = std::fs::remove_file(&report_path);
            }
        }
    }

    // Fallback: WMI BatteryStaticData / BatteryFullChargedCapacity
    let mut ps = std::process::Command::new("powershell");
    ps.args([
        "-NoProfile",
        "-NonInteractive",
        "-Command",
        "$d = (Get-CimInstance -Namespace root\\WMI -ClassName BatteryStaticData -ErrorAction SilentlyContinue | Measure-Object -Property DesignedCapacity -Sum).Sum; if (-not $d -or $d -eq 0) { $d = (Get-CimInstance -Namespace root\\WMI -ClassName BatteryFullChargedCapacity -ErrorAction SilentlyContinue | Measure-Object -Property FullChargedCapacity -Sum).Sum }; if ($d) { [int]$d }",
    ]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        ps.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    if let Ok(out) = ps.output() {
        if out.status.success() {
            let txt = String::from_utf8_lossy(&out.stdout);
            if let Ok(mwh) = txt.trim().parse::<u64>() {
                if mwh > 0 {
                    return Some(mwh_to_wh(mwh));
                }
            }
        }
    }

    None
}

static CACHED_BATTERY_WH: std::sync::OnceLock<Option<u32>> = std::sync::OnceLock::new();

pub fn get_battery_info() -> (String, u8, bool) {
    // 1. Detect physical battery capacity in Wh via powercfg /batteryreport or WMI (Cached via OnceLock)
    let wh_opt = *CACHED_BATTERY_WH.get_or_init(detect_battery_capacity_wh);
    if let Some(wh) = wh_opt {
        let mut percent = 100;
        let mut is_charging = true;
        unsafe {
            let mut power_status: SYSTEM_POWER_STATUS = std::mem::zeroed();
            if GetSystemPowerStatus(&mut power_status) != 0 {
                percent = if power_status.BatteryLifePercent <= 100 {
                    power_status.BatteryLifePercent
                } else {
                    100
                };
                is_charging = power_status.ACLineStatus == 1;
            }
        }
        return (format!("Batería {} Wh", wh), percent, is_charging);
    }

    // 2. Fallback to Win32 power status API (defaulting to Wh display)
    unsafe {
        let mut power_status: SYSTEM_POWER_STATUS = std::mem::zeroed();
        if GetSystemPowerStatus(&mut power_status) != 0 {
            let percent = if power_status.BatteryLifePercent <= 100 {
                power_status.BatteryLifePercent
            } else {
                100
            };
            let is_charging = power_status.ACLineStatus == 1;
            return ("Batería 50 Wh".to_string(), percent, is_charging);
        }
    }
    ("Batería 50 Wh".to_string(), 100, true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_battery_wh_from_xml_standard() {
        let xml = r#"<BatteryReport>
            <Batteries>
                <Battery>
                    <DesignCapacity>42000</DesignCapacity>
                    <FullChargeCapacity>41200</FullChargeCapacity>
                </Battery>
            </Batteries>
        </BatteryReport>"#;
        assert_eq!(parse_battery_wh_from_xml(xml), Some(42));
    }

    #[test]
    fn test_parse_battery_wh_from_xml_nested_capacity() {
        let xml = r#"<BatteryReport>
            <RuntimeEstimates>
                <DesignCapacity>
                    <Capacity>56000</Capacity>
                </DesignCapacity>
            </RuntimeEstimates>
        </BatteryReport>"#;
        assert_eq!(parse_battery_wh_from_xml(xml), Some(56));
    }

    #[test]
    fn test_parse_battery_wh_from_xml_empty_desktop() {
        let xml = r#"<BatteryReport>
            <Batteries></Batteries>
            <RuntimeEstimates>
                <DesignCapacity><Capacity>0</Capacity></DesignCapacity>
            </RuntimeEstimates>
        </BatteryReport>"#;
        assert_eq!(parse_battery_wh_from_xml(xml), None);
    }
}
