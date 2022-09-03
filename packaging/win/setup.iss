; Script generated by the Inno Setup Script Wizard.
; SEE THE DOCUMENTATION FOR DETAILS ON CREATING INNO SETUP SCRIPT FILES!
#include "environment.iss"

[Setup]
AppName=chim
AppVersion=0.1.0
AppPublisher=Jeff Dickey
AppSupportURL=https://chim.sh/
DefaultDirName={autopf}\chim
SourceDir=..\..
OutputBaseFilename=chim-setup
OutputDir=dist
AppId=chim
ChangesEnvironment=True

[Files]
Source: "target\release\chim.exe"; DestDir: "{app}\bin"
Source: "README.md"; DestDir: "{app}"; Flags: isreadme

[Tasks]
Name: envPath; Description: "Add to PATH variable" 

[Code]
procedure CurStepChanged(CurStep: TSetupStep);
begin
    if CurStep = ssPostInstall 
     then EnvAddPath(ExpandConstant('{app}') +'\bin');
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
    if CurUninstallStep = usPostUninstall
    then EnvRemovePath(ExpandConstant('{app}') +'\bin');
end;