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

// ed25519库 github.com/cometbft/cometbft/crypto/ed25519
// ed25519 节点Validator私钥 44da02ea3d3829415ff1175467c5f1cf9e3b4b90ef740758e2d9bccbb2520b1971492d9da0d7c2f82bc28b18ee17a34a58656963e022cf1d43143ca788f81510
// 公钥 71492d9da0d7c2f82bc28b18ee17a34a58656963e022cf1d43143ca788f81510
// 卫星用这个 公钥就可以找到卫星，卫星列表 有公钥信息
func TestAddr(t *testing.T) {
	tmPriv := "44da02ea3d3829415ff1175467c5f1cf9e3b4b90ef740758e2d9bccbb2520b1971492d9da0d7c2f82bc28b18ee17a34a58656963e022cf1d43143ca788f81510"
	b, _ := hex.DecodeString(tmPriv)
	priv := ed25519.PrivKey(b)
	pubkey := priv.PubKey()
	fmt.Println("pubkey: ", hex.EncodeToString(pubkey.Bytes()))
	address := priv.PubKey().Address()
	fmt.Println("原始地址", address)

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

		fmt.Println("Bench32地址", i, bech32Addr)
	}

	msg := []byte("needsignmessage")
	h := sha256.Sum256(msg)
	signData, _ := priv.Sign(h[:])
	fmt.Println("签名数据:", hex.EncodeToString(signData))

	// 公钥 验证
	// 需要 msg 以及公钥
	// 对msg 进行一次hash操作
	r := pubkey.VerifySignature(h[:], signData)
	fmt.Println("签名数据验证结果:", r)
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
