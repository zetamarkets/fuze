import * as anchor from '@project-serum/anchor';
import { Program } from '@project-serum/anchor';
import { VaultPutSell } from '../target/types/vault_put_sell';

describe('vault-put-sell', () => {

  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.Provider.env());

  const program = anchor.workspace.VaultPutSell as Program<VaultPutSell>;

  it('Is initialized!', async () => {
    // Add your test here.
    const tx = await program.rpc.initialize({});
    console.log("Your transaction signature", tx);
  });
});
