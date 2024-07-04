package test_code

import (
	"crypto/sha256"
	"encoding/hex"
	"fmt"
	"testing"

	"github.com/cometbft/cometbft/crypto/ed25519"
	"github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/bech32"
)

// ed25519 lib github.com/cometbft/cometbft/crypto/ed25519
// ed25519 private_key 44da02ea3d3829415ff1175467c5f1cf9e3b4b90ef740758e2d9bccbb2520b1971492d9da0d7c2f82bc28b18ee17a34a58656963e022cf1d43143ca788f81510
// public_key 71492d9da0d7c2f82bc28b18ee17a34a58656963e022cf1d43143ca788f81510
func TestAddr(t *testing.T) {
	tmPriv := "44da02ea3d3829415ff1175467c5f1cf9e3b4b90ef740758e2d9bccbb2520b1971492d9da0d7c2f82bc28b18ee17a34a58656963e022cf1d43143ca788f81510"
	b, _ := hex.DecodeString(tmPriv)
	priv := ed25519.PrivKey(b)
	pubkey := priv.PubKey()
	fmt.Println("pubkey: ", hex.EncodeToString(pubkey.Bytes()))
	address := priv.PubKey().Address()
	fmt.Println("original address", address)

	prefix := "space"
	conf := types.GetConfig()
	conf.SetBech32PrefixForAccount(prefix, prefix+"pub")
	conf.SetBech32PrefixForValidator(prefix+"valoper", prefix+"valoperpub")
	conf.SetBech32PrefixForConsensusNode(prefix+"valcons", prefix+"valconspub")
	bech32Prefixes := bech32Prefixes(conf)
	// space addresses
	for i, prefix := range bech32Prefixes {
		bech32Addr, err := bech32.ConvertAndEncode(prefix, address.Bytes())
		if err != nil {
			panic(err)
		}

		fmt.Println("Bench32 address", i, bech32Addr)
	}

	msg := []byte("needsignmessage")
	h := sha256.Sum256(msg)
	signData, _ := priv.Sign(h[:])
	fmt.Println("signature:", hex.EncodeToString(signData))

	r := pubkey.VerifySignature(h[:], signData)
	fmt.Println("verification result:", r)
}

func bech32Prefixes(config *types.Config) []string {
	return []string{
		config.GetBech32AccountAddrPrefix(),
		config.GetBech32AccountPubPrefix(),
		config.GetBech32ValidatorAddrPrefix(),
		config.GetBech32ValidatorPubPrefix(),
		config.GetBech32ConsensusAddrPrefix(),
		config.GetBech32ConsensusPubPrefix(),
	}
}
