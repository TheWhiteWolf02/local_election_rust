FROM rust:latest

RUN apt update && apt install -y gcc

RUN mkdir /local_election_rust

ADD ./src /local_election_rust/src/
ADD ./Cargo.toml /local_election_rust/
ADD ./Cargo.lock /local_election_rust/
ADD ./build.sh /local_election_rust/

WORKDIR /local_election_rust

CMD sh build.sh
