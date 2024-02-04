use encoding_rs::GBK;
use std::io::Write;
use tokio::io::AsyncReadExt;
use tokio_serial::{SerialPort, SerialPortBuilderExt, SerialStream};

pub struct SNR9816 {
    serial: SerialStream,
}

impl SNR9816 {
    #[allow(dead_code)]
    pub fn new(port: String, baudrate: u32) -> Self {
        SNR9816 {
            serial: tokio_serial::new(port, baudrate)
                .open_native_async()
                .unwrap(),
        }
    }

    fn _write(&mut self, data: Vec<u8>) {
        self.serial
            .write_all(&data)
            .expect("Failed to write to serial port");
    }

    // 6.3.1 tts, ok=0x41
    fn _tts(&self, text: String) -> Vec<u8> {
        let encoded = GBK.encode(text.as_str()).0.to_vec(); // 使用encoding_rs进行编码
        let length = encoded.len() + 2;
        let mut result = Vec::with_capacity(length + 5);
        result.push(0xfd); // head

        result.push((length >> 8) as u8); // len[15:8]
        result.push(length as u8); // len[7:0]
                                   // warn!("length={}", length);
                                   // println!("text={}", text);
                                   // println!("encoded={:?}", encoded);
                                   // println!("length={}", length);

        result.push(0x01); // command
        result.push(0x01); // codec

        result.extend_from_slice(&encoded);
        result
    }

    // 6.3.2.1 check status, idle=0x4f, busy=0x4e
    fn _check_status(&self) -> Vec<u8> {
        Vec::from([0xfd, 0x00, 0x01, 0x21])
    }

    // 6.3.2.2 pause, ok=0x41
    fn _pause(&self) -> Vec<u8> {
        Vec::from([0xfd, 0x00, 0x01, 0x03])
    }

    // 6.3.2.3 stop, ok=0x41
    fn _stop(&self) -> Vec<u8> {
        Vec::from([0xfd, 0x00, 0x01, 0x02])
    }

    // 6.3.3 volume, speed, tone, ok=0x41
    fn _config(&self, config: String) -> Vec<u8> {
        let mut result = Vec::with_capacity(9);
        result.extend_from_slice(&[0xfd, 0x00, 0x06, 0x01, 0x01]);
        result.extend_from_slice(&GBK.encode(config.as_str()).0.to_vec());
        result
    }

    async fn config(&mut self, text: String) {
        // self.wait_idle().await;
        self._write(self._config(format!("[{}]", text)))
    }

    #[allow(dead_code)]
    pub async fn volume(&mut self, value: u8) {
        self.config(format!("v{}", value)).await
    }

    #[allow(dead_code)]
    pub async fn speed(&mut self, value: u8) {
        self.config(format!("s{}", value)).await
    }

    #[allow(dead_code)]
    pub async fn tone(&mut self, value: u8) {
        self.config(format!("t{}", value)).await
    }

    // 6.3.4 ring, ok=0x41
    async fn _notify(&mut self, config: String) {
        self.tts(config).await
    }

    #[allow(dead_code)]
    pub async fn ring(&mut self, id: u8) {
        self._notify(format!("ring_{}", id).to_string()).await
    }

    #[allow(dead_code)]
    pub async fn message(&mut self, id: u8) {
        self._notify(format!("message_{}", id).to_string()).await
    }

    #[allow(dead_code)]
    pub async fn alert(&mut self, id: u8) {
        self._notify(format!("alert_{}", id).to_string()).await
    }

    async fn is_busy(&mut self) -> bool {
        let _ = self.serial.flush();
        let _ = self.serial.clear(tokio_serial::ClearBuffer::Input);
        let _ = self._write(self._check_status());
        let mut reader = tokio::io::BufReader::new(&mut self.serial);

        let mut buffer = [0; 1];
        let my_duration = tokio::time::Duration::from_millis(10);
        let _ = tokio::time::timeout(my_duration, reader.read(&mut buffer)).await;

        println!("busy={}", buffer[0]);
        if buffer[0] == 0x4f {
            return false;
        } else {
            return true;
        }
    }

    async fn wait_idle(&mut self) {
        while self.is_busy().await {
            std::thread::sleep(tokio::time::Duration::from_millis(100));
        }
    }

    #[allow(dead_code)]
    pub async fn tts(&mut self, text: String) {
        self.wait_idle().await;
        self._write(self._tts(text))
    }
}
