import type { 
  Instruction,
  Rpc,
  TransactionPartialSigner,
  TransactionMessage,
  TransactionMessageWithFeePayer,
  TransactionMessageWithBlockhashLifetime,
} from '@solana/kit';
import {
  pipe,
  createTransactionMessage,
  setTransactionMessageFeePayerSigner,
  setTransactionMessageLifetimeUsingBlockhash,
  appendTransactionMessageInstructions,
} from '@solana/kit';

export type TransactionSigner = TransactionPartialSigner<string>;

/**
 * Safely stringify objects that may contain BigInt values
 */
export function safeStringify(obj: any): string {
  return JSON.stringify(obj, (key, value) => {
    if (typeof value === 'bigint') {
      return `BigInt(${value.toString()})`;
    }
    return value;
  }, 2);
}

export async function buildTransaction(
  rpc: Rpc<any>,
  payer: TransactionSigner,
  instructions: Instruction[]
): Promise<{
  transactionMessage: TransactionMessage & TransactionMessageWithFeePayer<string> & TransactionMessageWithBlockhashLifetime;
  latestBlockhash: { blockhash: string; lastValidBlockHeight: bigint };
}> {
  const { value: latestBlockhash } = await (rpc as any).getLatestBlockhash().send();
  
  if (!latestBlockhash) {
    throw new Error('Failed to get latest blockhash');
  }

  if (instructions.length === 0) {
    throw new Error('Transaction has no instructions');
  }

  const transactionMessage = pipe(
    createTransactionMessage({ version: 0 }),
    (tx) => setTransactionMessageFeePayerSigner(payer, tx),
    (tx) => setTransactionMessageLifetimeUsingBlockhash(latestBlockhash, tx),
    (tx) => appendTransactionMessageInstructions(instructions, tx)
  );

  return { transactionMessage, latestBlockhash };
}
