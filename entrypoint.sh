#!/bin/bash

USER_ID=${LOCAL_UID:-9001}
GROUP_ID=${LOCAL_GID:-9001}

echo "Starting with UID: $USER_ID, GID: $GROUP_ID"
useradd -u $USER_ID -o -m user
groupmod -g $GROUP_ID user
export HOME=/home/user

id

# GitHub packages authToken
if [ ! -e /app/.authtoken ]; then
  echo "NOTE: If you want to publish the package, you need an .authtoken file."
  echo "Alternatively, you can add the credentials to ~/.npmrc ."
else
  cp /app/.authtoken /home/user/.npmrc
  echo "[INFO] Copied an .authtoken file to ~/.npmrc ."
fi

echo "$@"
exec /usr/sbin/gosu user "$@"
