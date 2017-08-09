#!/bin/sh
# @Author: BlahGeek
# @Date:   2017-07-15
# @Last Modified by:   BlahGeek
# @Last Modified time: 2017-08-09

echo "
[{
    \"title\": \"$(curl http://whatismyip.akamai.com/)\",
    \"icon\": \"character:FontAwesome:\"
}]
"
