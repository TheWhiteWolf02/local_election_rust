FROM rust:1.67

RUN apt update && apt install -y gcc

RUN mkdir /local_election_rust

ADD ./src /local_election_rust/src/
ADD ./Cargo.toml /local_election_rust/
ADD ./Cargo.lock /local_election_rust/
ADD ./build.sh /local_election_rust/

WORKDIR /local_election_rust

EXPOSE 24000

ENV EMMC_ADDRESS="0.0.0.0:0000"

CMD sh build.sh
