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
| Triggers | ❌ | ❌
| Select + Start | ✅ | ✅
| Joysick press | ✅ | ✅
## Usage
### Prerequisites
#### Linux
- Make the program executable: `sudo chmod a+x under-control-ler`
### Hosting
#### Linux
`./under-control-ler host [port]`
#### Windows
`.\under-control-ler.exe host [port]`
### Joining
#### Linux
`./under-control-ler join <address> [port]`
#### Windows
`.\under-control-ler.exe join <address> [port]`
## Compatibility
### Hosting
- Hosting is currently only possible on linux and spawns a virtual Xbox 360 gamepad using uinput
### Joining
- I only own an Xbox one gamepad but all gamepads supporting Xinput on windows and evdev on linux should work
