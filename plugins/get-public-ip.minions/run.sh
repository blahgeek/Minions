#!/bin/bash

echo $(curl -s ipv4.whatismyip.akamai.com) &
echo $(curl -s ipv6.whatismyip.akamai.com) &
wait
