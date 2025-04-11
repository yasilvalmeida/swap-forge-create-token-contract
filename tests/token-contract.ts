import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { TokenContract } from '../target/types/token_contract';
import { PublicKey, Keypair, Transaction } from '@solana/web3.js';
import { assert } from 'chai';

const TOKEN_METADATA_PROGRAM_ID = new PublicKey(
  'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'
);

describe('token-contract', () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.TokenContract as Program<TokenContract>;
  const wallet = provider.wallet;

  it('Creates a new token with metadata', async () => {
    // Create test data
    const name = 'TikTok Elon Dance';
    const symbol = 'ELINDA';
    const decimals = 9;
    const initialSupply = new anchor.BN(1000000000);
    const uri =
      'https://coffee-defensive-hare-118.mypinata.cloud/ipfs/bafkreihxjp3wqvmnvtuqgicoyx6whie2x7uh4p3cnj4hjpmmsjoandufve';

    // Generate keypair for mint
    const mintKeypair = Keypair.generate();

    try {
      // Derive PDAs
      const [metadata] = await PublicKey.findProgramAddressSync(
        [
          Buffer.from('metadata'),
          TOKEN_METADATA_PROGRAM_ID.toBuffer(),
          mintKeypair.publicKey.toBuffer(),
        ],
        TOKEN_METADATA_PROGRAM_ID
      );

      // Build the transaction
      const tx = await program.methods
        .createToken(
          name,
          symbol,
          decimals,
          uri,
          initialSupply,
          true,
          true,
          true
        )
        .accounts({
          payer: wallet.publicKey,
          mint: mintKeypair.publicKey,
          metadata,
        })
        .transaction();

      // Set transaction parameters
      tx.feePayer = wallet.publicKey;
      tx.recentBlockhash = (await provider.connection.getRecentBlockhash()).blockhash;
      
      // Sign the transaction
      tx.sign(wallet.payer!, mintKeypair);

      // Send and confirm the transaction
      const rawTransaction = tx.serialize();
      const txSignature = await provider.connection.sendRawTransaction(rawTransaction);
      
      const latestBlockHash = await provider.connection.getLatestBlockhash();
      await provider.connection.confirmTransaction({
        signature: txSignature,
        blockhash: latestBlockHash.blockhash,
        lastValidBlockHeight: latestBlockHash.lastValidBlockHeight,
      });

      console.log('Token creation transaction signature:', txSignature);
      assert.exists(txSignature);

      // Verify mint account was created
      const mintAccount = await provider.connection.getAccountInfo(mintKeypair.publicKey);
      assert.isNotNull(mintAccount, 'Mint account was not created');
    } catch (error) {
      console.error('Error creating token:', error);
      throw error;
    }
  });
});