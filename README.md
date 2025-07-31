# Introduction

This project is a TCP port multiplexer for Minecraft servers, allowing multiple servers to share a single port. It is designed to improve server management and reduce the need for multiple open ports.

This is a study project and is not intended for production use. It is a work in progress and may contain bugs or incomplete features. If you are looking for a stable solution, please consider using established alternatives.

# Usage

1. Install it using `cargo install mc-tcpmux`.
2. Create a configuration file based on `config.example.toml`. Instructions can be found inside the file.
3. Run the server with `mc-tcpmux path/to/config.toml`. Add `-r` to auto-reload the configuration file on changes.

# Technical Details

[The handshake packet](https://minecraft.wiki/w/Java_Edition_protocol/Packets#Handshake) sent by the client contains the server address and port used to connect. This project intercepts that packet and proxies the connection to the appropriate server based on the configuration file.

# Similar Projects

- [Connor14/MinecraftServerProxy](https://github.com/Connor14/MinecraftServerProxy)
- [RenegadeEagle/minecraft-redirect-proxy](https://github.com/RenegadeEagle/minecraft-redirect-proxy)
- [Ktlo/MCSHub]( https://github.com/Ktlo/MCSHub)
- [handtruth/mcshub](https://github.com/handtruth/mcshub)
- [janispritzkau/minecraft-reverse-proxy](https://github.com/janispritzkau/minecraft-reverse-proxy)
