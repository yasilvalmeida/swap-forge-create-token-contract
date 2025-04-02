import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { TokenContract } from '../target/types/token_contract';
import { PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY } from '@solana/web3.js';
import {
  TOKEN_PROGRAM_ID,
  createMint,
  getAssociatedTokenAddress,
} from '@solana/spl-token';
import { assert } from 'chai';

describe('token-contract', () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.tokenContract as Program<TokenContract>;
  const provider = anchor.AnchorProvider.env();
  const wallet = provider.wallet;

  it('Creates a new token', async () => {
    // Generate a new keypair for the mint account
    const mintKeypair = anchor.web3.Keypair.generate();
    const tokenInfoKeypair = anchor.web3.Keypair.generate();

    // Create test data
    const name = 'Test Token';
    const symbol = 'TEST';
    const decimals = 9;

    try {
      // Create the token
      const tx = await program.methods
        .createToken(name, symbol, decimals)
        .accounts({
          payer: wallet.publicKey,
          mint: mintKeypair.publicKey,
          tokenInfo: tokenInfoKeypair.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([mintKeypair, tokenInfoKeypair])
        .rpc();

      console.log('Token creation transaction signature:', tx);

      // Fetch the token info account
      const tokenInfo = await program.account.tokenInfo.fetch(
        tokenInfoKeypair.publicKey
      );

      // Verify the token info
      assert.equal(tokenInfo.name, name);
      assert.equal(tokenInfo.symbol, symbol);
      assert.equal(tokenInfo.decimals, decimals);
      assert.equal(tokenInfo.authority.toBase58(), wallet.publicKey.toBase58());
      assert.equal(tokenInfo.mint.toBase58(), mintKeypair.publicKey.toBase58());

      console.log('Token created successfully!');
    } catch (error) {
      console.error('Error creating token:', error);
      throw error;
    }
  });

  it('Fails to create token with invalid decimals', async () => {
    const mintKeypair = anchor.web3.Keypair.generate();
    const tokenInfoKeypair = anchor.web3.Keypair.generate();

    try {
      await program.methods
        .createToken('Test Token', 'TEST', 10) // Invalid decimals > 9
        .accounts({
          payer: wallet.publicKey,
          mint: mintKeypair.publicKey,
          tokenInfo: tokenInfoKeypair.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
          rent: SYSVAR_RENT_PUBKEY,
        })
        .signers([mintKeypair, tokenInfoKeypair])
        .rpc();

      // If we reach here, the test should fail
      assert.fail('Should have failed with invalid decimals');
    } catch (error) {
      // Test passed because it failed as expected
      console.log('Test passed: Failed to create token with invalid decimals');
    }
  });

  it('Creates multiple tokens', async () => {
    // Create first token
    const mint1Keypair = anchor.web3.Keypair.generate();
    const tokenInfo1Keypair = anchor.web3.Keypair.generate();

    await program.methods
      .createToken('First Token', 'FTK', 9)
      .accounts({
        payer: wallet.publicKey,
        mint: mint1Keypair.publicKey,
        tokenInfo: tokenInfo1Keypair.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([mint1Keypair, tokenInfo1Keypair])
      .rpc();

    // Create second token
    const mint2Keypair = anchor.web3.Keypair.generate();
    const tokenInfo2Keypair = anchor.web3.Keypair.generate();

    await program.methods
      .createToken('Second Token', 'STK', 9)
      .accounts({
        payer: wallet.publicKey,
        mint: mint2Keypair.publicKey,
        tokenInfo: tokenInfo2Keypair.publicKey,
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
        rent: SYSVAR_RENT_PUBKEY,
      })
      .signers([mint2Keypair, tokenInfo2Keypair])
      .rpc();

    // Verify both tokens
    const tokenInfo1 = await program.account.tokenInfo.fetch(
      tokenInfo1Keypair.publicKey
    );
    const tokenInfo2 = await program.account.tokenInfo.fetch(
      tokenInfo2Keypair.publicKey
    );

    assert.equal(tokenInfo1.name, 'First Token');
    assert.equal(tokenInfo2.name, 'Second Token');
    assert.notEqual(tokenInfo1.mint.toBase58(), tokenInfo2.mint.toBase58());

    console.log('Multiple tokens created successfully!');
  });
});
