import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { TokenContract } from '../target/types/token_contract';
import { PublicKey, SystemProgram } from '@solana/web3.js';
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createAssociatedTokenAccountInstruction,
  getAssociatedTokenAddress,
  TOKEN_PROGRAM_ID,
} from '@solana/spl-token';
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
    const name = 'Trump Tarif EU';
    const symbol = 'TTEU';
    const decimals = 9;
    const initialSupply = new anchor.BN(1000000000);
    const uri =
      'https://coffee-defensive-hare-118.mypinata.cloud/ipfs/bafkreihxjp3wqvmnvtuqgicoyx6whie2x7uh4p3cnj4hjpmmsjoandufve';

    // Generate keypair for mint
    const mintKeypair = anchor.web3.Keypair.generate();

    try {
      // Derive the metadata account address
      const [metadata] = await PublicKey.findProgramAddressSync(
        [
          Buffer.from('metadata'),
          TOKEN_METADATA_PROGRAM_ID.toBuffer(),
          mintKeypair.publicKey.toBuffer(),
        ],
        TOKEN_METADATA_PROGRAM_ID
      );
      // Create the token
      const tx = await program.methods
        .createToken(name, symbol, decimals, uri, initialSupply)
        .accounts({
          payer: wallet.publicKey,
          mint: mintKeypair.publicKey,
          metadata,
          sysvarInstructions: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
          tokenMetadataProgram: TOKEN_METADATA_PROGRAM_ID,
        })
        .remainingAccounts([
          {
            pubkey: SystemProgram.programId,
            isWritable: false,
            isSigner: false,
          },
          {
            pubkey: anchor.web3.SYSVAR_RENT_PUBKEY,
            isWritable: false,
            isSigner: false,
          },
          {
            pubkey: TOKEN_PROGRAM_ID,
            isWritable: false,
            isSigner: false,
          },
        ])
        .signers([mintKeypair])
        .rpc();

      console.log('Token creation transaction signature:', tx);
      assert.exists(tx);
    } catch (error) {
      console.error('Error creating token:', error);
      throw error;
    }
  });
});
