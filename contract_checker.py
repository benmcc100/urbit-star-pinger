import urlock
from web3 import Web3
import sys
import mysql.connector
import time
import json

# get star names from azimuth ids
with open('pre.txt', 'r') as file:
    prefixes = file.read()
with open('suf.txt', 'r') as file:
    suffixes = file.read()

# connect to contract
w3 = Web3(Web3.HTTPProvider('https://mainnet.infura.io/v3/API_KEY'))

address = '0x223c067F8CF28ae173EE5CafEa60cA44C335fecB' # star contract address
with open('abi.json', 'r') as file:
    abi = file.read()
contract_instance = w3.eth.contract(address=address, abi=abi)

stars_to_update = {}

latest_block = w3.eth.get_block('latest')['number']
start_block = str(hex(latest_block - 720)) # 2 hours of blocks

spawn_filter = contract_instance.events.Spawned.createFilter(fromBlock=start_block, toBlock="latest") #, argument_filters={'event': 'Spawned'})
spawns = spawn_filter.get_all_entries()
for spawn in spawns:
    star = spawn['args']['prefix']
    stars_to_update[star] = 1      

sponsor_change_filter = contract_instance.events.EscapeAccepted.createFilter(fromBlock=start_block, toBlock="latest") #, argument_filters={'event': 'Spawned'})
changes = sponsor_change_filter.get_all_entries()
for change in changes:
    star = change['args']['sponsor']
    stars_to_update[star] = 1

sponsor_loss_filter = contract_instance.events.LostSponsor.createFilter(fromBlock=start_block, toBlock="latest") #, argument_filters={'event': 'Spawned'})
changes = sponsor_loss_filter.get_all_entries()
for change in changes:
    star = change['args']['sponsor']
    stars_to_update[star] = 1

print(stars_to_update)

# connect to mysql server
db = mysql.connector.connect(
    host="localhost",
    user="USERNAME",
    password="PASSWORD",
    database="urbit_star_data"
)

cursor = db.cursor()

add_star = "UPDATE stars SET sponsoring_count = %s, planets_spawned = %s WHERE point_int = %s"
for i in stars_to_update.keys():
    star = i
    pre = 3*(i // 256)
    suf = 3*(i % 256)
    star_name = prefixes[pre:pre+3] + suffixes[suf:suf+3]

    sponsoring_count = contract_instance.functions.getSponsoringCount(star).call()
    planets_spawned = contract_instance.functions.getSpawnCount(star).call()

    star_data = (sponsoring_count, planets_spawned, i)

    cursor.execute(add_star, star_data)
    print(star_name)
    db.commit()
    time.sleep(0.2)

# disconnect to mysql server
cursor.close()
db.close()
