#!/bin/sh
set -eu

lsblk -d -P -o NAME,MODEL,SERIAL,SIZE,ROTA,TYPE,TRAN,VENDOR,REV
