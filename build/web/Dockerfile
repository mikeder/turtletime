# Serves the web build of the game.
# Build the game with `trunk build --release` first, then from the repository root:
#   docker build -f build/web/Dockerfile -t turtletime .
FROM nginx:alpine
COPY build/web/nginx.conf /etc/nginx/conf.d/default.conf
COPY dist/ /usr/share/nginx/html/
# compress the big files once here instead of on every request
RUN gzip -k -9 /usr/share/nginx/html/*.wasm /usr/share/nginx/html/*.js
