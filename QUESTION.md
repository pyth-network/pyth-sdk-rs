
To get started, use the short "Cmd + Shift + V" to preview the markdown. Alternatively, click on the preview button on the top right corner.

## Question 
Do you see the time condition here? 
I think if we change status to unknown make sure that prev_* is the aggregate which is an added overhead in every chain. What do you think?

I again looped back at only storing "latest trading price" as current price. 

### Code Snippets

[pyth-sdk-solana/src/state.rs#L343](pyth-sdk-solana/src/state.rs#L343)	
````
    pub fn to_price_feed(&self, price_key: &Pubkey) -> PriceFeed {
        #[allow(unused_mut)]
        let mut status = self.agg.status;

        #[cfg(target_arch = "bpf")]
        if matches!(status, PriceStatus::Trading)
            && Clock::get().unwrap().slot - self.agg.pub_slot > VALID_SLOT_PERIOD
        {
            status = PriceStatus::Unknown;
        }

        PriceFeed::new(
            price_key.to_bytes(),
            status,
            self.timestamp as u64,
            self.expo,
            self.num,
            self.num_qt,
            self.prod.val,
            self.agg.price,
            self.agg.conf,
            self.ema_price.val,
            self.ema_conf.val as u64,
            self.prev_price,
            self.prev_conf,
            self.prev_timestamp as u64
        )
    }

````

### Terminal Output
````

````
	