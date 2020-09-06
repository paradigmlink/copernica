use {
    crate::{Transport, decode, encode},
    copernica::{
        InterLinkPacket, Link, ReplyTo
    },
    anyhow::{anyhow, Result},
    crossbeam_channel::{Sender, Receiver, unbounded},
};

pub struct MpscCorruptor {
    link: Link,
    // t = tansport; c = copernic; 0 = this instance of t; 1 = the pair of same type
    t2c_tx: Sender<InterLinkPacket>,
    c2t_rx: Receiver<InterLinkPacket>,
    t2t0_tx: Sender<Vec<u8>>,        // give
    t2t0_rx: Receiver<Vec<u8>>,      // keep
    t2t1_tx: Option<Vec<Sender<Vec<u8>>>>,
}

impl MpscCorruptor {
    pub fn male(&self) -> Sender<Vec<u8>> {
        self.t2t0_tx.clone()
    }
    pub fn female(&mut self, new_t2t1_tx: Sender<Vec<u8>>) {
        if let None = self.t2t1_tx {
            self.t2t1_tx = Some(vec![]);
        }
        if let Some(t2t1_tx) = &mut self.t2t1_tx {
            t2t1_tx.push(new_t2t1_tx);
        }
    }
}

impl<'a> Transport<'a> for MpscCorruptor {
    fn new(link: Link
        , (t2c_tx, c2t_rx): ( Sender<InterLinkPacket> , Receiver<InterLinkPacket> )
        ) -> Result<MpscCorruptor> {
        match link.reply_to() {
            ReplyTo::Mpsc => {
                let (t2t0_tx, t2t0_rx) = unbounded::<Vec<u8>>();
                return Ok(
                    MpscCorruptor {
                        link,
                        t2c_tx,
                        c2t_rx,
                        t2t0_tx,
                        t2t0_rx,
                        t2t1_tx: None,
                    })
            }
            _ => return Err(anyhow!("MpscCorruptor Transport expects a LinkId of type LinkId::Mpsc")),
        }
    }

    #[allow(unreachable_code)]
    fn run(&self) -> Result<()> {
        let link = self.link.clone();
        let t2t0_rx = self.t2t0_rx.clone();
        let t2c_tx = self.t2c_tx.clone();
        std::thread::spawn(move || {
            match link.reply_to() {
                ReplyTo::Mpsc => {
                    loop {
                        match t2t0_rx.recv(){
                            Ok(msg) => {
                                let nw = decode(msg)?;
                                let ilp = InterLinkPacket::new(link.clone(), nw);
                                let _r = t2c_tx.send(ilp)?;
                            },
                            Err(error) => return Err(anyhow!("{}", error)),
                        };
                    }
                },
                _ => {},
            }
            Ok::<(), anyhow::Error>(())
        });

        let c2t_rx = self.c2t_rx.clone();
        if let Some(t2t1_tx) = self.t2t1_tx.clone() {
            std::thread::spawn(move || {
                loop {
                    match c2t_rx.recv(){
                        Ok(ilp) => {
                            let nw = encode(ilp)?;
                            let mut corrupted = nw;
                            for i in 4..10 {
                                corrupted[i] = 0x0;
                            }
                            for s in t2t1_tx.clone() {
                                s.send(corrupted.clone())?;
                            }
                        },
                        Err(error) => return Err(anyhow!("{}", error)),
                    }
                }
                Ok::<(), anyhow::Error>(())
            });
        } else {
            return Err(anyhow!("You need to bind the transports before using them, i.e. t0.female(t1.male()); followed by: t1.female(t0.male());"))
        }

        Ok(())
    }
}

