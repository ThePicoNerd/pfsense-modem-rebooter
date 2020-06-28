#!/bin/bash

iface="wifi0"
plug="10.0.0.93"

grace_period=30

# encoded (the reverse of decode) commands to send to the plug

# encoded {"system":{"set_relay_state":{"state":1}}}
payload_on="AAAAKtDygfiL/5r31e+UtsWg1Iv5nPCR6LfEsNGlwOLYo4HyhueT9tTu36Lfog=="

# encoded {"system":{"set_relay_state":{"state":0}}}
payload_off="AAAAKtDygfiL/5r31e+UtsWg1Iv5nPCR6LfEsNGlwOLYo4HyhueT9tTu3qPeow=="

send_to_plug() {
  port="9999"
  payload="$1"
  if ! echo -n "$payload" | base64 --decode | nc $plug $port -q 1; then
    echo couldn''t connect to $plug:$port, nc failed with exit code $?
  fi
}

switch_on() {
  send_to_plug $payload_on
}

switch_off() {
  send_to_plug $payload_off
}

reboot() {
  echo "Cutting modem power."
  switch_off

  sleep 3

  echo "Starting modem ..."
  switch_on

  echo "Waiting $grace_period seconds for modem to boot."
  sleep $grace_period
}

iface_is_up() {
  echo ifconfig "$iface" | grep "UP"
  return 0
}

echo "Hit [CTRL+C] to stop"

while :; do
  if iface_is_up; then
  sleep 5
  else
  reboot
  fi
done
