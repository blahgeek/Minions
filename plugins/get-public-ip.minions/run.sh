#!/bin/sh
# @Author: BlahGeek
# @Date:   2017-07-15
# @Last Modified by:   BlahGeek
# @Last Modified time: 2017-07-15

echo "
{
    \"results\": [{
        \"title\": \"$(curl http://whatismyip.akamai.com/)\",
        \"icon\": \"character:FontAwesome:\"
    }]
}
"
