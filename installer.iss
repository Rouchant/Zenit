; Script de Instalación para Zenit Showroom
; Requiere Inno Setup (https://jrsoftware.org/isdl.php)

#define MyAppName "Zenit Showroom"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "Zenit"
#define MyAppExeName "zenit-showroom.exe"

[Setup]
AppId={{D37E6081-3051-40F0-B00B-882200E4B1F8}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
OutputDir=dist
OutputBaseFilename=ZenitShowroom-Setup-v{#MyAppVersion}
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequired=admin
SetupIconFile=assets\logo.ico

[Languages]
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
; Ejecutable principal compilado en release
Source: "target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
; Librería libmpv obligatoria para reproducción de video
Source: "libmpv-2.dll"; DestDir: "{app}"; Flags: ignoreversion
; Carpeta de recursos y videos
Source: "assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs createallsubdirs
; Scripts de configuración y detección de hardware
Source: "system-setup.ps1"; DestDir: "{app}"; Flags: ignoreversion
Source: "get-specs.ps1"; DestDir: "{app}"; Flags: ignoreversion

[Registry]
; Inicio automático en el Registro de Windows (HKLM Run) obligatorio para modo Kiosco
Root: HKLM; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "{#MyAppName}"; ValueData: """{app}\{#MyAppExeName}"""; Flags: uninsdeletevalue

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Tasks: desktopicon
; Acceso directo en la carpeta de Inicio de Windows (Startup común) automático
Name: "{commonstartup}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"

[Run]
; Ejecución obligatoria y automática de configuración de energía, batería, suspensión y brillo máximo
Filename: "powershell.exe"; Parameters: "-ExecutionPolicy Bypass -NoProfile -WindowStyle Hidden -File ""{app}\system-setup.ps1"""; StatusMsg: "Configurando energía, batería, suspensión y brillo máximo al 100%..."; Flags: runhidden waituntilterminated
; Iniciar Zenit automáticamente al terminar de instalar sin checkboxes ni preguntas
Filename: "{app}\{#MyAppExeName}"; Flags: nowait
