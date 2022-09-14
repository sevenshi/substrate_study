import React, { useState,useEffect } from 'react'
import { Form, Grid } from 'semantic-ui-react'

import { TxButton } from './substrate-lib/components'
import { useSubstrateState } from './substrate-lib'

import KittyCards from './KittyCards';

const parseKitty = (index,{ dna, price, owner }) => ({
  id:index,
  dna,
  price: price.toJSON(),
  owner: owner.toJSON(),
})

export default function Kitties (_props) {
  const { currentAccount } = useSubstrateState()
  const { api, keyring } = useSubstrateState()
  const  accountPair  = currentAccount;
  const [kittyCnt, setKittyCnt] = useState(0)
  const [kitties, setKitties] = useState([]);
  const [status, setStatus] = useState('')

  useEffect(() => {
    let unsubscribe
    api.query.kittiesModule
      .kittiesCount(newValue => {
        setKittyCnt (newValue.toNumber())
      })
      .then(unsub => {
        unsubscribe = unsub
      })
      .catch(console.error)

    return () => unsubscribe && unsubscribe()
  }, [api.query.kittiesModule.kittiesCount, keyring])

  useEffect(() => {
    const kittyIndices = [...Array(kittyCnt).keys()];
    let unsubscribe
    api.query.kittiesModule.kitties.multi(
      kittyIndices,
      kitties => {
        const kittiesMap = kitties.map((kitty,index) => parseKitty(index,kitty.unwrap()))
        setKitties(kittiesMap)
      }
    )
      .then(unsub => {
        unsubscribe = unsub
      })
      .catch(console.error)

    return () => unsubscribe && unsubscribe()
  }, [api.query.kittiesModule.kitties,kitties, keyring])

  console.log(kitties)

  return <Grid.Column width={16}>
    <h1>小毛孩 Count: <span>{kittyCnt}</span></h1>
    <KittyCards kitties={kitties} accountPair={accountPair} setStatus={setStatus}/>

    <Form style={{ margin: '1em 0' }}>
      <Form.Field style={{ textAlign: 'center' }}>
        <TxButton
          label="领养"
          type='SIGNED-TX'
          setStatus={setStatus}
          attrs={{
            palletRpc: 'kittiesModule',
            callable: 'create',
            inputParams: [],
            paramFields: []
          }}
        />
      </Form.Field>
    </Form>
    <div style={{ overflowWrap: 'break-word' }}>{status}</div>
  </Grid.Column>

}
