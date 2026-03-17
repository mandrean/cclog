FROM alpine:3

RUN apk add --no-cache git

ARG TARGETARCH
COPY ${TARGETARCH}/cclog /usr/local/bin/cclog

ENTRYPOINT ["cclog"]
