FROM nginx:stable-alpine

COPY configuration/instantiated/reverse-proxy/index.html /var/www/index.html
COPY configuration/instantiated/reverse-proxy/default.conf /etc/nginx/conf.d
COPY configuration/instantiated/reverse-proxy/.htpasswd /etc/nginx/.htpasswd
