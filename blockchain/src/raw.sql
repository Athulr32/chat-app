INSERT INTO "NativeTokenBalance" VALUES ($1, $2,$3) ON CONFLICT(public_key) DO UPDATE SET "token_balance" = $4;