const web3 = require("@solana/web3.js");
const {struct, b, u8, u32} = require("@solana/buffer-layout");

const contract = web3.Keypair.fromSecretKey(new Uint8Array([223,143,8,70,205,100,4,197,222,158,132,43,89,182,188,243,24,213,136,120,189,209,235,13,167,45,132,41,17,243,58,158,114,230,85,178,27,22,80,213,200,96,166,64,152,163,191,112,35,197,55,219,24,254,117,129,227,39,37,232,106,30,178,193]))

export const invoke = async (loan: string, collateral: string) => {
    if (contract.publicKey != process.argv[2]) {
        console.info("Please update the contract keypair in invoke.ts with build/example_sol_contract-keypair.json.");
        return;
    }
    console.info("Invoking contract " + contract.publicKey);
    let conn = new web3.Connection(web3.clusterApiUrl('devnet'));

    /* Prepare the payer account */
    console.info("Airdropping to payer account...");
    let payer = web3.Keypair.generate();
    let airdropSig = await conn.requestAirdrop(
        payer.publicKey, web3.LAMPORTS_PER_SOL
    );
    await conn.confirmTransaction(airdropSig);

    /* Prepare the createInst instruction: Create an account to store the
     * LoanInfo data, which will be passed to Init for initialization */
    let loanInfoSize = 1 + 32 + 8 + 32 + 8;
    let dataAccount = web3.Keypair.generate();
    let cost = await conn.getMinimumBalanceForRentExemption(loanInfoSize);
    const createInst = web3.SystemProgram.createAccount({
        lamports: cost,
        space: loanInfoSize,
        programId: contract.publicKey,
        fromPubkey: payer.publicKey,
        newAccountPubkey: dataAccount.publicKey,
    });

    /* Specify the accounts and parameters for invocations */
    const dataKey = dataAccount.publicKey;
    const loanKey = new web3.PublicKey(loan);
    const collateralKey = new web3.PublicKey(collateral);
    let accounts =
        [{pubkey: contract.publicKey, isSigner: true, isWritable: false},
         {pubkey: dataKey, isSigner: false, isWritable: false},
         {pubkey: loanKey, isSigner: false, isWritable: false},
         {pubkey: collateralKey, isSigner: false, isWritable: false},
        ];
    let dataLayout = struct([ u8('instruction') ])
    let data = Buffer.alloc(dataLayout.span);
    dataLayout.encode(Object.assign({instruction: 0}), data);

    /* Invoke the Init instruction (instruction #0) */
    console.log("Creating data account and invoking Init...");
    let txInit = new web3.Transaction({ feePayer: payer.publicKey });
    txInit.add(
        createInst, 
        new web3.TransactionInstruction({
            data: data,
            keys: accounts,
            programId: contract.publicKey
        })
    );
    let txInitSig = await web3.sendAndConfirmTransaction(conn, txInit, [payer, dataAccount, contract]);
    console.log("TxHash: " + txInitSig);

    /* Invoke the Loan2Value instruction (instruction #1) */
    console.log("Checking loan to value ratio...");
    let txCheck = new web3.Transaction({ feePayer: payer.publicKey });
    dataLayout.encode(Object.assign({instruction: 1}), data);
    txCheck.add(
        new web3.TransactionInstruction({
            data: data,
            keys: accounts,
            programId: contract.publicKey
        })
    );
    let txCheckSig = await web3.sendAndConfirmTransaction(conn, txCheck, [payer, contract]);
    console.log("TxHash: " + txCheckSig);    

    /* Try to invoke the Init instruction without authority */
    console.log("Trying an unauthorized invocation of Init...");
    let attacker = web3.Keypair.generate();
    accounts[0].pubkey = attacker.publicKey

    let attackerDataAccount = web3.Keypair.generate();
    const attackerCreateInst = web3.SystemProgram.createAccount({
        lamports: cost,
        space: loanInfoSize,
        programId: contract.publicKey,
        fromPubkey: payer.publicKey,
        newAccountPubkey: attackerDataAccount.publicKey,
    });

    let txAttacker = new web3.Transaction({ feePayer: payer.publicKey });
    dataLayout.encode(Object.assign({instruction: 0}), data);
    txAttacker.add(
        attackerCreateInst, 
        new web3.TransactionInstruction({
            data: data,
            keys: accounts,
            programId: contract.publicKey
        })
    );
    let txAttackerSig = await web3.sendAndConfirmTransaction(conn, txAttacker, [payer, attackerDataAccount, attacker]);
    console.log("TxHash: " + txAttackerSig);
}

let ethToUSD = "EdVCmQ9FSPcVe5YySXDPCRmc8aDQLKJ9xvYBMZPie1Vw";
let usdtToUSD = "38xoQ4oeJCBrcVvca2cGk7iV1dAfrmTR1kmhSCJQ8Jto";
invoke(ethToUSD, usdtToUSD);
