import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { DecentralizedAutonomousCivilization } from "../target/types/decentralized_autonomous_civilization";

describe("decentralized-autonomous-civilization", () => {
  // Configure the client to use the local cluster.
  anchor.setProvider(anchor.AnchorProvider.env());

  const program = anchor.workspace.decentralizedAutonomousCivilization as Program<DecentralizedAutonomousCivilization>;

  it("Is initialized!", async () => {
    // Add your test here.
    const tx = await program.methods.initialize().rpc();
    console.log("Your transaction signature", tx);
  });
});
