concordium-client contract invoke 12265 --subindex 0 \
  --entrypoint get_admin \
  --grpc-ip grpc.testnet.concordium.com \
  --grpc-port 20000 \
  --secure

  concordium-client contract update 12265 --subindex 0 \
  --entrypoint register \
  --sender my-ccd \
  --energy 40000 \
  --parameter-json register.json \
  --grpc-ip grpc.testnet.concordium.com \
  --grpc-port 20000 \
  --secure


  concordium-client contract invoke 12265 --subindex 0 \
  --entrypoint is_verified \
  --parameter-json check_subject.json \
  --grpc-ip grpc.testnet.concordium.com \
  --grpc-port 20000 \
  --secure


  concordium-client contract invoke 12265 --subindex 0 \
  --entrypoint get_commitment \
  --parameter-json check_subject.json \
  --grpc-ip grpc.testnet.concordium.com \
  --grpc-port 20000 \
  --secure


  concordium-client contract invoke 12265 --subindex 0 \
  --entrypoint is_idp \
  --parameter-json check_idp.json \
  --grpc-ip grpc.testnet.concordium.com \
  --grpc-port 20000 \
  --secure


  concordium-client contract update 12265 --subindex 0 \
  --entrypoint use_nullifier \
  --sender my-ccd \
  --energy 40000 \
  --parameter-json use_nullifier.json \
  --grpc-ip grpc.testnet.concordium.com \
  --grpc-port 20000 \
  --secure


  concordium-client contract invoke 12265 --subindex 0 \
  --entrypoint nullifier_used \
  --parameter-json check_nullifier.json \
  --grpc-ip grpc.testnet.concordium.com \
  --grpc-port 20000 \
  --secure


  concordium-client contract update 12265 --subindex 0 \
  --entrypoint revoke \
  --sender my-ccd \
  --energy 40000 \
  --parameter-json revoke.json \
  --grpc-ip grpc.testnet.concordium.com \
  --grpc-port 20000 \
  --secure


  concordium-client contract invoke 12265 --subindex 0 \
  --entrypoint get_revoked_at \
  --parameter-json check_subject.json \
  --grpc-ip grpc.testnet.concordium.com \
  --grpc-port 20000 \
  --secure


  concordium-client transaction status <TX_HASH> \
  --grpc-ip grpc.testnet.concordium.com \
  --grpc-port 20000 \
  --secure



  DEMO:




  concordium-client contract invoke 12265 --subindex 0 \
  --entrypoint is_verified \
  --parameter-json check_subject.json \
  --grpc-ip grpc.testnet.concordium.com \
  --grpc-port 20000 \
  --secure


  concordium-client contract invoke 12265 --subindex 0 \
  --entrypoint get_commitment \
  --parameter-json check_subject.json \
  --grpc-ip grpc.testnet.concordium.com \
  --grpc-port 20000 \
  --secure


  concordium-client contract update 12265 --subindex 0 \
  --entrypoint use_nullifier \
  --sender my-ccd \
  --energy 40000 \
  --parameter-json use_nullifier_demo.json \
  --grpc-ip grpc.testnet.concordium.com \
  --grpc-port 20000 \
  --secure



  concordium-client contract invoke 12265 --subindex 0 \
  --entrypoint nullifier_used \
  --parameter-json check_nullifier_demo.json \
  --grpc-ip grpc.testnet.concordium.com \
  --grpc-port 20000 \
  --secure



  concordium-client transaction status 62f4d8a6688376e08a44b04301d1142e1017050e9542691db3fbbb5f17d7023f \
  --grpc-ip grpc.testnet.concordium.com \
  --grpc-port 20000 \
  --secure