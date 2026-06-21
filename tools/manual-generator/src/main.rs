use char;

fn main() {
    for i in 0..=0x10FFFF{
       match char::from_u32(i) {
            Some(ch) => {
                let mut utf8buf =  [0;4];
                ch.encode_utf8(&mut utf8buf);

                let binary = format!("{:08b} {:08b} {:08b} {:08b}", utf8buf[0], utf8buf[1], utf8buf[2], utf8buf[3]);
                println!("{:06X}:{}:{}",i,binary,ch);
            },
            None => {}
       }
    }
}
