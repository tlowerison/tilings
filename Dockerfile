FROM rust:1.52.1 AS build-rust

  RUN rustup target add wasm32-unknown-unknown
  RUN [ -z "$(which wasm-pack)" ] && cargo install wasm-pack

  WORKDIR /app

  RUN mkdir -p www
  RUN mkdir -p www/pkg
  COPY rust rust

  WORKDIR /app/rust/wasm

  RUN wasm-pack build --out-dir ../../www/pkg --release

FROM node:12.20.2 as build-js

  RUN apt-get update && apt-get install -y jq

  WORKDIR /app/www

  RUN mkdir -p pkg
  COPY www/package.json www/yarn.lock ./
  RUN yarn --ignore-engines
  COPY --from=build-rust /app/www/pkg ./pkg
  RUN yarn add ./pkg --ignore-engines
  RUN yarn --ignore-engines
  COPY www ./
  RUN yarn build

FROM nginx:1.21.0

  COPY nginx /etc/nginx
  COPY --from=build-js /app/www/dist /usr/share/nginx/html
  RUN ROOT=/usr/share/nginx/html envsubst < /etc/nginx/nginx.conf.template | sed -e 's/§/$/g' > /etc/nginx/nginx.conf

  EXPOSE 3000
  CMD ["nginx", "-g", "daemon off;"]