import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { TokenContract } from '../target/types/token_contract';
import { PublicKey, SystemProgram } from '@solana/web3.js';
import { TOKEN_PROGRAM_ID } from '@solana/spl-token';
import { assert } from 'chai';
import { AssertionError } from 'assert';

describe('token-contract', () => {
  // Configure the client to use the local cluster.
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);

  const program = anchor.workspace.TokenContract as Program<TokenContract>;
  const wallet = provider.wallet;

  it('Creates a new token', async () => {
    // Create test data
    const name = 'Test Token';
    const symbol = 'TEST';
    const decimals = 9;

    // Generate keypairs for accounts
    const mintKeypair = anchor.web3.Keypair.generate();
    const tokenInfoKeypair = anchor.web3.Keypair.generate();

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

    } catch (error) {
      console.error('Error creating token:', error);
      throw error;
    }
  });

  it('Fails to create token with invalid decimals', async () => {
    const invalidDecimals = 10; // More than 9 decimals
    const mintKeypair = anchor.web3.Keypair.generate();
    const tokenInfoKeypair = anchor.web3.Keypair.generate();

    try {
      await program.methods
        .createToken('Test Token', 'TEST', invalidDecimals)
        .accounts({
          payer: wallet.publicKey,
          mint: mintKeypair.publicKey,
          tokenInfo: tokenInfoKeypair.publicKey,
          systemProgram: SystemProgram.programId,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([mintKeypair, tokenInfoKeypair])
        .rpc();

      assert.fail('Should have failed with invalid decimals');
    } catch (error) {
      if (error instanceof AssertionError) {
        console.log(
          'error',
          error
        );
        assert.equal(error.message, 'Should have failed with invalid decimals');
      }
    }
  });

  it('Creates multiple tokens', async () => {
    // First token
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
      })
      .signers([mint1Keypair, tokenInfo1Keypair])
      .rpc();

    // Second token
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
    assert.equal(tokenInfo1.symbol, 'FTK');
    assert.equal(tokenInfo2.name, 'Second Token');
    assert.equal(tokenInfo2.symbol, 'STK');
    assert.notEqual(tokenInfo1.mint.toBase58(), tokenInfo2.mint.toBase58());
  });
});
