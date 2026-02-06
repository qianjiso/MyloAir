import * as nodeCrypto from 'crypto';

export interface CryptoAdapter {
  encrypt(text: string): string;
  decrypt(text: string): string;
}

export function createCrypto(encryptionKey: string): CryptoAdapter {
  const key = nodeCrypto.pbkdf2Sync(encryptionKey, 'salt', 10000, 32, 'sha256');
  return {
    encrypt(text: string): string {
      const iv = nodeCrypto.randomBytes(16);
      const cipher = nodeCrypto.createCipheriv('aes-256-cbc', key, iv);
      let encrypted = cipher.update(text, 'utf8', 'hex');
      encrypted += cipher.final('hex');
      return iv.toString('hex') + ':' + encrypted;
    },
    decrypt(encryptedText: string): string {
      try {
        const parts = encryptedText.split(':');
        if (parts.length !== 2) return encryptedText;
        const iv = Buffer.from(parts[0], 'hex');
        const encrypted = parts[1];
        const decipher = nodeCrypto.createDecipheriv('aes-256-cbc', key, iv);
        let decrypted = decipher.update(encrypted, 'hex', 'utf8');
        decrypted += decipher.final('utf8');
        return decrypted;
      } catch {
        return encryptedText;
      }
    }
  };
}

