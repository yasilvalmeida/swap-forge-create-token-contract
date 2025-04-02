import fs from 'fs';
import path from 'path';
import bs58 from 'bs58';

// Read the Solana CLI keypair file
const keypairPath = path.join(
  require('os').homedir(),
  '.config',
  'solana',
  'id.json'
);
const keypairFile = fs.readFileSync(keypairPath, 'utf-8');
const secretKey = Uint8Array.from(JSON.parse(keypairFile));

// Convert to Base58 (for Phantom/Solflare)
const privateKeyBase58 = bs58.encode(secretKey);
console.log('Private Key (Base58):', privateKeyBase58);

// Or keep it as a raw byte array (for Backpack)
console.log('Private Key (Bytes):', Array.from(secretKey));
