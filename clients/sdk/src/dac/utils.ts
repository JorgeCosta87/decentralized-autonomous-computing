import {
  type TransactionSigner,
  type SolanaClient,
  type Instruction,
  createTransaction,
  signTransactionMessageWithSigners,
  getSignatureFromTransaction,
  sendAndConfirmTransactionFactory,
} from 'gill';

export async function sendTransaction(
  client: SolanaClient,
  payer: TransactionSigner,
  instructions: Instruction[]
): Promise<string> {
  const { value: latestBlockhash } = await client.rpc.getLatestBlockhash().send();

  const transaction = createTransaction({
    feePayer: payer.address,
    instructions,
    latestBlockhash,
  });

  const signedTransaction = await signTransactionMessageWithSigners(transaction);

  const signature = getSignatureFromTransaction(signedTransaction);

  const sendAndConfirmTransaction = sendAndConfirmTransactionFactory({ 
    rpc: client.rpc,
    rpcSubscriptions: client.rpcSubscriptions,
  });
  await sendAndConfirmTransaction(signedTransaction, { commitment: 'confirmed' });

  return signature;
}
