cargo build
copy appx\* .\target\debug
cd .\target\debug
powershell -command "Get-AppxPackage *under-control-ler* | Remove-AppxPackage"
powershell -command "Add-AppxPackage -Register AppxManifest.xml"
cd ..\..\
