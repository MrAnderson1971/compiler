```
# Install vcpkg if you don't have it
git clone https://github.com/microsoft/vcpkg.git
cd vcpkg
.\bootstrap-vcpkg.bat

# Integrate with Visual Studio
.\vcpkg.exe integrate install

# Install Keystone and Unicorn
.\vcpkg.exe install keystone:x64-windows
.\vcpkg.exe install unicorn:x64-windows


```
