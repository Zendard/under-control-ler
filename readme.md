# Under Control(ler)
Share gamepad inputs peer-to-peer (WIP)
## Progress
| Item | Linux | Windows
|-------|-------|-------
| Hosting | ✅ | ❌
| Joining | ✅ | ✅
| ABXY | ✅ | ✅
| D-pad | ✅ | ✅
| Joysticks (left+right) | ✅ | ✅
| Bumpers | ✅ | ✅
| Triggers | ✅ | ✅
| Select + Start | ✅ | ✅
| Joysick press | ✅ | ✅
## Usage
### Hosting
#### Linux
- Make the program executable: `sudo chmod a+x under-control-ler`
- Start hosting: `./under-control-ler host [port]`
#### Windows
`.\under-control-ler.exe host [port]`
### Joining
#### Linux
- Make the program executable: `sudo chmod a+x under-control-ler`
- Join: `./under-control-ler join <address> [port]`
#### Windows
`.\under-control-ler.exe join <address> [port]`
## Compatibility
### Hosting
- Hosting is currently only possible on linux as it spawns a virtual Xbox 360 gamepad using uinput
### Joining
- I only own an Xbox one gamepad but all gamepads supporting Xinput on windows and evdev on linux should work
- Use [DS4Windows](https://github.com/Ryochan7/DS4Windows/releases) for Playstation controllers
