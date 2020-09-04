use {
    crate::{Transport},
    copernica::{
        InterLinkPacket, Link, ReplyTo
    },
    anyhow::{anyhow, Result},
    crossbeam_channel::{Sender, Receiver, unbounded},
};

pub struct MpscChannel {
    link: Link,
    // t = tansport; c = copernic; 0 = this instance of t; 1 = the pair of same type
    t2c_tx: Sender<InterLinkPacket>,
    c2t_rx: Receiver<InterLinkPacket>,
    t2t0_tx: Sender<InterLinkPacket>,        // give
    t2t0_rx: Receiver<InterLinkPacket>,      // keep
    t2t1_tx: Option<Vec<Sender<InterLinkPacket>>>,
}

impl MpscChannel {
    pub fn male(&self) -> Sender<InterLinkPacket> {
        self.t2t0_tx.clone()
    }
    pub fn female(&mut self, new_t2t1_tx: Sender<InterLinkPacket>) {
        if let None = self.t2t1_tx {
            self.t2t1_tx = Some(vec![]);
        }
        if let Some(t2t1_tx) = &mut self.t2t1_tx {
            t2t1_tx.push(new_t2t1_tx);
        }
    }
}

impl<'a> Transport<'a> for MpscChannel {
    fn new(link: Link
        , (t2c_tx, c2t_rx): ( Sender<InterLinkPacket> , Receiver<InterLinkPacket> )
        ) -> Result<MpscChannel> {
        match link.reply_to() {
            ReplyTo::Mpsc => {
                let (t2t0_tx, t2t0_rx) = unbounded::<InterLinkPacket>();
                return Ok(
                    MpscChannel {
                        link,
                        t2c_tx,
                        c2t_rx,
                        t2t0_tx,
                        t2t0_rx,
                        t2t1_tx: None,
                    })
            }
            _ => return Err(anyhow!("MpscChannel Transport expects a LinkId of type LinkId::Mpsc")),
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
                                let _r = t2c_tx.send(msg.change_origination(link.clone()))?;
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
                            for s in t2t1_tx.clone() {
                                s.send(ilp.clone())?;
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

