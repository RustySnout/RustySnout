# RustySnout

A Rust app for network monitoring and control 
Spring 2024


## Setting up development env

This is a [tauri](https://tauri.app/) App with [Next,js](https://nextjs.org/) front end

To run 
- clone the repo
- run `npm install`
- run `npm run tauri dev`

### monitoring not working?

- terminate app
- run in `src-tauri` this command 
```
sudo setcap cap_sys_ptrace,cap_dac_read_search,cap_net_raw,cap_net_admin+ep target/debug/rustysnout
```
- run in root `npm run tauri dev`

## project structure

The main entry point is `src-tauri/src/main.rs` which also is the Rust backend

Next.JS frontend resides in `src/`
The main entry point for the frontend is in `src/app/page.js`

