FROM rust:latest

WORKDIR /usr/src/tank-game
COPY . .

RUN apt-get update && apt-get install -y libwayland-client0 librust-wayland-client-dev librust-alsa-sys-dev librust-libudev-dev libxkbcommon0 mesa-vulkan-drivers
RUN cargo install --path .

CMD ["tank-game"]
