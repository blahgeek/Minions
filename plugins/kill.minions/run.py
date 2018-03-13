#!/usr/bin/env python3
# -*- coding: utf-8 -*-
# @Author: BlahGeek
# @Date:   2017-08-12
# @Last Modified by:   BlahGeek
# @Last Modified time: 2018-03-13

import json
import psutil


def display_process(process):
    return {
        'title': ' '.join(process.cmdline()),
        'subtitle': '{}, CPU {:.1f}%, MEM {:.1f}%, {}'
                    .format(process.pid, process.cpu_percent(),
                            process.memory_percent(), process.status()),
        'action': 'kill {}'.format(process.pid),
    }


if __name__ == '__main__':
    processes = list(psutil.process_iter())
    processes.sort(key=lambda p: (p.cpu_percent(), p.memory_percent()),
                   reverse=True)
    print(json.dumps([display_process(p) for p in processes],
                     indent=4))
