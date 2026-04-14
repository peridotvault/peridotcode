/**
 * Main entry point for {{game_title}}
 * Sets up the Phaser game configuration
 */

import BootScene from './scenes/BootScene.js';
import GameScene from './scenes/GameScene.js';

const config = {
    type: Phaser.AUTO,
    width: 800,
    height: 600,
    parent: 'game-container',
    backgroundColor: '#2d2d44',
    physics: {
        default: 'arcade',
        arcade: {
            gravity: { y: 0 },
            debug: false
        }
    },
    scene: [BootScene, GameScene]
};

// Create the game instance
const game = new Phaser.Game(config);

console.log('{{game_title}} initialized');
