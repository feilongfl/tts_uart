# tts_uart

实现一个rust程序，打开一个串口和Actix http服务器。
当GET访问/时，打印OK
当GET访问/v1/tts/snr9816时，打印OK
当POST访问/v1/tts/snr9816时，将urlencode和utf-8编码的text字段，转换为gb2312后通过串口发送
通过clap解析命令行参数（参数应放在一个struct中,使用command宏定义），设置串口,波特率，服务器地址和端口
请给出完整代码和cargo依赖安装命令，要求程序尽可能简短
