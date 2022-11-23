FROM ubuntu:20.04

COPY ./target/release/node-template ./usr/bin

EXPOSE 30333 9944 9933

ENTRYPOINT ["node-template"]
