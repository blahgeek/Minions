#! /usr/bin/env python
# -*- coding: utf-8 -*-

import sys
import json
from asteval import Interpreter


expr = sys.argv[1]
print(json.dumps([{
    'title': str(Interpreter()(expr)),
    'subtitle': expr,
}]))
