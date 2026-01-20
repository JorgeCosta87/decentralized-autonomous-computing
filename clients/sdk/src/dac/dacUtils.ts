import type { Address } from '@solana/kit';

/**
 * Decode base64 string to Uint8Array
 */
function decodeBase64(base64String: string): Uint8Array {
  return new Uint8Array(Buffer.from(base64String, 'base64'));
}

/**
 * Decode accounts from getProgramAccounts response
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
        data: decodeBase64(String(account.data)),
        executable: account.executable,
        lamports: account.lamports,
        programAddress: account.owner,
        space: account.space ?? 0n,
      }),
    );

    return decoded.map((item) => item.data);
  } catch (error: any) {
    console.error('Error decoding accounts', error);
    return [];
  }
}
