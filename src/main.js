import { GameScene } from './scenes/GameScene.js';
import { BootScene } from './scenes/BootScene.js';

const config = {
    type: Phaser.AUTO,
    width: 800,
    height: 600,
    parent: 'game-container',
    backgroundColor: '#87CEEB',
    physics: {
        default: 'arcade',
        arcade: {
            gravity: { y: 300 },
            debug: false
        }
    },
    scene: [BootScene, GameScene]
};

const game = new Phaser.Game(config);

console.log('peridotcode started!');