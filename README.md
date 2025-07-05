# Ant Upload

Share your files with the world — one drag and drop to eternal access.

Censorship-proof, universally available, and free for everyone.

Pay once to upload — downloads stay free forever.

Liberate the world's knowledge — give it back to everyone

![Ant Upload](./assets/ant_upload.gif)

## Download it

[Download the latest release on github with a click!](https://github.com/grumbach/ant_upload/releases/latest) 

Or download directly from the Autonomi Network:

```bash
# macOS aarch64
ant file download 86a59dabc6d2d0f9cb50c03859a39378ce59e5f2e849aee2094eff641b21d438 AntUpload-aarch64-apple-darwin.zip

# Linux aarch64
ant file download 4b114c0749a5e24a8d5b3e6f6b60bd6ce245b635583cd8a200e10c5385d2b9d2 AntUpload-aarch64-unknown-linux-musl.zip

# macOS x86_64 
ant file download 97725e3a9a65ca88a8975aa0d1f84d3d23a6cb7fa25c4861936574e0b180a217 AntUpload-x86_64-apple-darwin.zip

# Linux x86_64
ant file download bf027cf6a05358c7731b8b1d18be5f7105005cfdd89881d327bd2e3b4a3d11d5 AntUpload-x86_64-unknown-linux-musl.zip
```

*AntUpload was uploaded to the Network using AntUpload!*

> Mac users might face quarantine issues: `"AntUpload.app" is damaged and can't be opened. You should move it to the Trash.`
>
> This happens because I don't have a $99 a year Apple Developer account :(
>
> To fix this:
> 1. **Unzip** the file (double-click the `.zip`).
> 2. Open **Terminal** (press `Cmd + Space`, type "Terminal", and press Enter).
> 3. Go to your Downloads folder:
>   ```bash
>   cd ~/Downloads
>   ```
> 4. Remove macOS quarantine flag:
>   ```bash
>   xattr -rd com.apple.quarantine AntUpload.app
>   ```
> 5. Double-click **AntUpload.app** to open it!

## Build it from source

```bash
# currently uses the bleeding edge of the autonomi API which will eventually be released but for avant-garde users here's a how to guide

# clone the autonomi repo and use the req_resp_record_put branch
git clone https://github.com/grumbach/autonomi.git 
cd autonomi
git fetch origin req_resp_record_put 
git checkout req_resp_record_put

# go back into the ant-upload directory
cd ..

# build the release version of the app
cargo build --release

# (for macOS) make a AntUpload.app
bash ./assets/mac_os_bundle.sh
```

## Run it from source

```bash
cargo run --release
```

## For those diving into the code

- The `src/server.rs` file contains the main logic for all autonomi network interaction
- The `src/main.rs` 90% AI vibe-coded front-end for the app
- The `src/cached_payments.rs` file is copy pasted as is from the ant CLI, it allows re-use of payments for retries (which means it's cross compatible with ant CLI)

## Coming soon

- Use the public Autonomi API release instead of the bleeding edge req_resp_record_put branch
- Windows binary releases (need help here)
- Download files from the Autonomi Network
- A repository of all shared files
- Suggest more features by submitting or upvoting an issue on github
