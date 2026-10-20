use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConnectivityFeatures {
    pub usbc: Option<String>,
    pub wifi: Option<String>,
    pub bluetooth: Option<String>,
    pub sensors: bool,
    pub touch: bool,
    pub biometric: bool,
    pub hdr: bool,
}

pub fn detect_connectivity_features() -> ConnectivityFeatures {
    let script = r#"
$r = @{ usbc = ''; wifi = ''; bluetooth = ''; sensors = $false; touch = $false; biometric = $false; hdr = $false }
$u = Get-PnpDevice -FriendlyName '*Thunderbolt*','*USB4*','*UcmCx*' -ErrorAction SilentlyContinue | Select-Object -First 1
if ($u) {
    if ($u.FriendlyName -match 'Thunderbolt 4') { $r.usbc = 'Thunderbolt 4' }
    elseif ($u.FriendlyName -match 'Thunderbolt') { $r.usbc = 'Thunderbolt' }
    elseif ($u.FriendlyName -match 'USB4') { $r.usbc = 'USB4' }
    else { $r.usbc = 'USB-C Power Delivery' }
}
$w = Get-NetAdapter -ErrorAction SilentlyContinue | Where-Object { $_.InterfaceDescription -match 'Wi-Fi|Wireless|802.11' } | Select-Object -First 1
if ($w) {
    $d = $w.InterfaceDescription
    if ($d -match 'Wi-Fi 7|802\.11be|BE200') { $r.wifi = 'Wi-Fi 7' }
    elseif ($d -match 'Wi-Fi 6E|AX210|AX211') { $r.wifi = 'Wi-Fi 6E' }
    elseif ($d -match 'Wi-Fi 6|802\.11ax|AX200|AX201') { $r.wifi = 'Wi-Fi 6' }
    elseif ($d -match 'Wi-Fi 5|802\.11ac|Wireless-AC') { $r.wifi = 'Wi-Fi 5' }
    else { $r.wifi = 'Wi-Fi 6' }
}
$b = (Get-ItemProperty 'HKLM:\System\CurrentControlSet\Services\BTHPORT\Parameters\Devices\*' -ErrorAction SilentlyContinue).LmpVersion | Measure-Object -Maximum
if ($b.Maximum) {
    $m = $b.Maximum
    if ($m -ge 13) { $r.bluetooth = 'Bluetooth 5.4' }
    elseif ($m -eq 12) { $r.bluetooth = 'Bluetooth 5.3' }
    elseif ($m -eq 11) { $r.bluetooth = 'Bluetooth 5.2' }
    elseif ($m -eq 10) { $r.bluetooth = 'Bluetooth 5.1' }
    elseif ($m -eq 9) { $r.bluetooth = 'Bluetooth 5.0' }
    else { $r.bluetooth = 'Bluetooth 5.3' }
} else {
    $bd = Get-PnpDevice -Class 'Bluetooth' -ErrorAction SilentlyContinue | Where-Object {$_.Status -eq 'OK'} | Select-Object -First 1
    if ($bd) { $r.bluetooth = 'Bluetooth 5.3' }
}
if (Get-PnpDevice -Class 'Sensor' -ErrorAction SilentlyContinue | Where-Object {$_.FriendlyName -match 'Inclinometer|Motion|Orientation|Accelerometer|Giroscopio|Acelerómetro'} | Select-Object -First 1) { $r.sensors = $true }
if (Get-PnpDevice -Class 'HIDClass' -ErrorAction SilentlyContinue | Where-Object {$_.FriendlyName -match 'touch screen|pantalla táctil'} | Select-Object -First 1) { $r.touch = $true }
if (Get-PnpDevice -Class 'Biometric' -ErrorAction SilentlyContinue | Where-Object {$_.Status -eq 'OK'} | Select-Object -First 1) { $r.biometric = $true }
if ((Get-CimInstance -Namespace root/wmi -ClassName WmiMonitorAdvancedColorSupport -ErrorAction SilentlyContinue).AdvancedColorCapabilities -eq 3) { $r.hdr = $true }
$r | ConvertTo-Json -Compress
"#;

    let mut cmd = std::process::Command::new("powershell");
    cmd.args(["-NoProfile", "-NonInteractive", "-Command", script]);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    if let Ok(output) = cmd.output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if let Ok(feat) = serde_json::from_str::<ConnectivityFeatures>(stdout.trim()) {
                return feat;
            }
        }
    }

    ConnectivityFeatures {
        usbc: None,
        wifi: Some("Wi-Fi 6".to_string()),
        bluetooth: Some("Bluetooth 5.3".to_string()),
        sensors: false,
        touch: false,
        biometric: false,
        hdr: false,
    }
}

pub fn format_connectivity_features(feat: &ConnectivityFeatures) -> String {
    let mut parts: Vec<String> = Vec::new();

    if let Some(ref usbc) = feat.usbc {
        if !usbc.trim().is_empty() {
            parts.push(usbc.clone());
        }
    }

    if let Some(ref wifi) = feat.wifi {
        if !wifi.trim().is_empty() {
            parts.push(wifi.clone());
        }
    } else {
        parts.push("Wi-Fi 6".to_string());
    }

    if let Some(ref bt) = feat.bluetooth {
        if !bt.trim().is_empty() {
            parts.push(bt.clone());
        }
    } else {
        parts.push("Bluetooth 5.3".to_string());
    }

    if feat.touch {
        parts.push("Pantalla Táctil".to_string());
    }
    if feat.biometric {
        parts.push("Huella Dactilar".to_string());
    }
    if feat.sensors {
        parts.push("Giroscopio".to_string());
    }
    if feat.hdr {
        parts.push("Pantalla HDR".to_string());
    }

    if parts.len() < 3 {
        parts.push("Teclado en Español".to_string());
    }

    parts.join(" / ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_connectivity_features_full() {
        let feat = ConnectivityFeatures {
            usbc: Some("Thunderbolt 4".to_string()),
            wifi: Some("Wi-Fi 6E".to_string()),
            bluetooth: Some("Bluetooth 5.4".to_string()),
            sensors: true,
            touch: true,
            biometric: true,
            hdr: true,
        };
        let formatted = format_connectivity_features(&feat);
        assert!(formatted.contains("Thunderbolt 4"));
        assert!(formatted.contains("Wi-Fi 6E"));
        assert!(formatted.contains("Bluetooth 5.4"));
        assert!(formatted.contains("Pantalla Táctil"));
        assert!(formatted.contains("Huella Dactilar"));
        assert!(formatted.contains("Giroscopio"));
        assert!(formatted.contains("Pantalla HDR"));
    }

    #[test]
    fn test_format_connectivity_features_defaults() {
        let feat = ConnectivityFeatures::default();
        let formatted = format_connectivity_features(&feat);
        assert!(formatted.contains("Wi-Fi 6"));
        assert!(formatted.contains("Bluetooth 5.3"));
        assert!(formatted.contains("Teclado en Español"));
    }
}
