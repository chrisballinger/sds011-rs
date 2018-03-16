# sds011-rs

[![Build Status](https://travis-ci.org/chrisballinger/sds011-rs.svg?branch=master)](https://travis-ci.org/chrisballinger/sds011-rs)

Rust crate for interacting with [SDS011](http://aqicn.org/sensor/sds011/) particle sensor. Inspired by [sds011_particle_sensor](https://gitlab.com/frankrich/sds011_particle_sensor) Python project.

This is my first Rust project, mostly as a learning exercise, so it's probably not very good or useful.


## Dependencies

* [Rust](https://www.rustup.rs/)
* [VS Code](https://code.visualstudio.com/) (optional)
  * [rls-vscode](https://github.com/rust-lang-nursery/rls-vscode)
  * [vscode-lldb](https://github.com/vadimcn/vscode-lldb)

#### macOS

CH34x driver: [CH341SER_MAC.ZIP](http://www.wch.cn/download/CH341SER_MAC_ZIP.html)
 
`sha256 b190f612b833727b2006f362a835f7e97177b580e45cef253e164202106c48eb`


## Running

CLI example

```
$ sds011 /dev/ttyUSB0 -o sensor_output/
```

systemd example `/etc/systemd/system/sds011.service`

```
[Unit]
Description=sds011

[Service]
ExecStart=/home/chip/sds011 /dev/ttyUSB0 -o /home/chip/sensor_output
Restart=always
User=chip
Group=chip

[Install]
WantedBy=multi-user.target
```

## Handling Output

Install `csvkit` to merge files.

```
$ pip3 install csvkit
$ cd sensor_output/
$ csvstack *.csv > joined.csv
```

Sample contents of `sensor_output` directory:

```
2017-12-18 01_04_02.csv2017-12-18 01_05_43.csv2017-12-18 01_07_24.csv2017-12-18 01_09_04.csv2017-12-18 01_10_45.csv2017-12-18 01_12_26.csv
```

Sample contents of `2017-12-18 01_02_21.csv`

```
timestamp,pm2_5,pm10
2017-12-18T01:02:22.412397014Z,1.6,9
2017-12-18T01:02:23.409393633Z,1.6,9.1
2017-12-18T01:02:24.407442938Z,1.6,9.2
2017-12-18T01:02:25.405230113Z,1.5,8.5
2017-12-18T01:02:26.403490463Z,1.5,8.5
2017-12-18T01:02:27.400479582Z,1.5,8.7
2017-12-18T01:02:28.398499803Z,1.4,7.6
2017-12-18T01:02:29.396514608Z,1.5,7.7
2017-12-18T01:02:30.394565413Z,1.5,7.7
```



## License

MIT