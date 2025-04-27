# Ant Upload

Share your files with the world — one drag and drop to eternal access.

Censorship-proof, universally available, and free for everyone.

Pay once to upload — downloads stay free forever.

Liberate the world's knowledge — give it back to everyone

![Ant Upload](./assets/ant_upload.gif)

## Build it

```bash
# currently uses the bleeding edge of the autonomi API which will eventually be released but for avant-garde users here's a how to guide

# clone the autonomi repo and use the client-light-networking branch
git clone https://github.com/maidsafe/autonomi.git 
cd autonomi
git fetch origin client-light-networking 
git checkout client-light-networking

# go back into the ant-upload directory
cd ..

# build the release version of the app
cargo build --release
```

## Run it

```bash
cargo run --release
```

## For those diving into the code

- The `src/server.rs` file contains the main logic for all autonomi network interaction
- The `src/main.rs` 90% AI vibe-coded front-end for the app

## Coming soon

- Use the public Autonomi API release instead of the bleeding edge client-light-networking branch
- Proper binary releases
- Download files from the Autonomi Network
- A repository of all shared files
- Suggest more features by submitting or upvoting an issue on github
