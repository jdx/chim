Set-PSDebug -Trace 1

Get-ChildItem target\release
New-Item dist\chim\bin -ItemType Directory -ea 0
Move-Item target\release\chim.exe dist\chim\bin\chim.exe
$Env:CHIM_VERSION = (cargo get version --pretty)
Compress-Archive -Path dist\chim -DestinationPath dist\chim-$env:CHIM_VERSION-windows-x64.zip
