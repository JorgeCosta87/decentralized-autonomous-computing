import type { Address } from 'gill';
import { base64 } from '@coral-xyz/anchor/dist/cjs/utils/bytes/index.js';

/**
 * Decode accounts from getProgramAccounts response.
 * Uses the working pattern: base64.decode(account.data.toString())
 */
export function decodeAccountsFromResponse<T>(
  response: Array<{ 
    pubkey: Address; 
    account: { 
      data: unknown; 
      executable: boolean; 
      owner: Address; 
      lamports: bigint; 
      space?: bigint;
    };
  }>,
  decodeFn: (encodedAccount: any) => { data: T }
): T[] {
  try {
    const decoded = response.map(({ pubkey, account }) =>
      decodeFn({
        address: pubkey,
        data: base64.decode(String(account.data)),
        executable: account.executable,
        lamports: account.lamports,
        programAddress: account.owner,
        space: account.space ?? 0n,
      }),
    );

    return decoded.map((item) => item.data);
  } catch (error: any) {
    return [];
  }
}
