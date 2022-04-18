FROM debian:bullseye
RUN apt-get update && apt install -y curl wget git make cpio gcc bc build-essential
WORKDIR /app
COPY . .

RUN bash kernel/mkkernel.sh \
    && bash rootfs/mkrootfs.sh

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y 
ENV PATH="/root/.cargo/bin:${PATH}"
RUN cargo build --release
