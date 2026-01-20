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

/**
 * Extract error message from simulation error
 */
export function extractSimulationError(simulation: any): string | null {
  if (!simulation?.value?.err) {
    return null;
  }

  const err = simulation.value.err;
  
  if (err.InstructionError) {
    const [instructionIndex, instructionError] = err.InstructionError;
    
    if (instructionError.Custom) {
      const errorCode = instructionError.Custom;
      if (errorCode === 1) {
        return 'MissingAccount (error code #1). This usually means one of the required accounts (goal, task, agent, or network_config) does not exist.';
      }
      return `Custom program error code: ${errorCode}`;
    }
    
    return `Instruction ${instructionIndex} error: ${safeStringify(instructionError)}`;
  }
  
  return safeStringify(err);
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
