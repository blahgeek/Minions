#!/usr/bin/env python3

import sys
import json
import datetime
import dateparser


items = []
dt = dateparser.parse(sys.argv[1], languages=['en', ])


def add(name, value):
    items.append({
        'title': str(value),
        'badge': name,
    })


if dt is not None:
    add('MSEC', int(dt.timestamp() * 1000))
    add('ISO', dt.astimezone().isoformat())
    add('UTC', dt.astimezone(datetime.timezone.utc).isoformat())
    add('CTIME', dt.ctime())
    add('SEC', int(dt.timestamp()))


print(json.dumps(items))
