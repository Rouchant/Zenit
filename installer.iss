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
ArchitecturesInstallIn64BitMode=x64
PrivilegesRequired=admin

[Languages]
Name: "spanish"; MessagesFile: "compiler:Languages\Spanish.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "autostart"; Description: "Iniciar Zenit Showroom automáticamente al encender la PC (Modo Exhibición)"; GroupDescription: "Configuración de Showroom:"; Flags: checkedonce

[Files]
; Ejecutable principal compilado en release
Source: "target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
; Librería libmpv obligatoria para reproducción de video
Source: "libmpv-2.dll"; DestDir: "{app}"; Flags: ignoreversion
; Carpeta de recursos y videos
Source: "assets\*"; DestDir: "{app}\assets"; Flags: ignoreversion recursesubdirs createallsubdirs
; Scripts auxiliares (opcional)
Source: "get-specs.ps1"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Tasks: desktopicon
; Acceso directo en la carpeta de Inicio de Windows si se seleccionó la tarea autostart
Name: "{commonstartup}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; WorkingDir: "{app}"; Tasks: autostart

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
