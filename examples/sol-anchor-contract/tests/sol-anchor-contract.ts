import * as anchor from "@coral-xyz/anchor";
import { SolAnchorContract } from "../target/types/sol_anchor_contract";

describe("sol-anchor-contract", () => {
  const fs = require("fs");
  const assert = require("assert");

  let ethToUSD = "EdVCmQ9FSPcVe5YySXDPCRmc8aDQLKJ9xvYBMZPie1Vw";
  let usdtToUSD = "38xoQ4oeJCBrcVvca2cGk7iV1dAfrmTR1kmhSCJQ8Jto";

  const provider = anchor.AnchorProvider.local();

  // Configure the client to use the local cluster.
  anchor.setProvider(provider);

  const config = anchor.web3.Keypair.generate();
  const program = anchor.workspace
    .SolAnchorContract as anchor.Program<SolAnchorContract>;
  const payer = provider.wallet.publicKey;
  let programId = program.programId;
  const programKey = program.programId;
  console.log(programKey)

  it("Initialize the config.", async () => {
    const tx = await program.methods
      .init({
        loanPriceFeedId: new anchor.web3.PublicKey(ethToUSD),
        collateralPriceFeedId: new anchor.web3.PublicKey(usdtToUSD),
      })
      .accounts({
        program: programId,
        payer,
        config: config.publicKey,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([config])
      .rpc();

    console.log("Config key: " + config.publicKey);
    console.log("Init() is invoked: " + tx);
  });

  it("Check loan to value ratio.", async () => {
    const tx = await program.methods
      .loanToValue(new anchor.BN(1), new anchor.BN(3000))
      .accounts({
        config: config.publicKey,
        pythLoanAccount: new anchor.web3.PublicKey(ethToUSD),
        pythCollateralAccount: new anchor.web3.PublicKey(usdtToUSD),
      })
      .signers([])
      .rpc();

    console.log("Loan2Value() is invoked: " + tx);
  });

  it("Prevent initialization of config without authority.", async () => {
    const attacker = anchor.web3.Keypair.generate();

    var tx,
      success = false;
    try {
      tx = await program.methods
        .init({
          loanPriceFeedId: new anchor.web3.PublicKey(ethToUSD),
          collateralPriceFeedId: new anchor.web3.PublicKey(usdtToUSD),
        })
        .accounts({
          program: programId,
          payer,
          config: config.publicKey,
          systemProgram: anchor.web3.SystemProgram.programId,
        })
        .signers([config, attacker])
        .rpc();

      success = true;
    } catch (error) {
      console.log("Attacker failed at unauthorized init");
    }
    if (success) throw new Error("Attacker succeeded! TxHash: " + tx);
  });
});
