#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# @Author: BlahGeek
# @Date:   2017-08-12
# @Last Modified by:   BlahGeek
# @Last Modified time: 2017-08-19

import sys
import json
import psutil


def display_process(process):
    return {
        'title': ' '.join(process.cmdline()),
        'subtitle': '{}, CPU {:.1f}%, MEM {:.1f}%, {}'
                    .format(process.pid, process.cpu_percent(),
                            process.memory_percent(), process.status()),
        'action_callback': ['run.py', str(process.pid)],
        'action_callback_returns': False,
    }


if __name__ == '__main__':
    if len(sys.argv) > 1:
        psutil.Process(int(sys.argv[1])).kill()
    else:
        processes = list(psutil.process_iter())
        processes.sort(key=lambda p: (p.cpu_percent(), p.memory_percent()),
                       reverse=True)
        print(json.dumps([display_process(p) for p in processes],
                         indent=4))
